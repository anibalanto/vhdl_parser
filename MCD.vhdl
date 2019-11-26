library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity MaximoComunDivisor is
generic(
		n : integer := 10
	);
	port(
		reloj	: in	std_logic;
		reset	: in	std_logic;
		pulsoa	: in	std_logic;
		pulsob	: in	std_logic;
		dato	: in	std_logic_vector( 10  downto  0 );
		fin	: out	std_logic;
		salida	: out	std_logic_vector( n-1  downto  0 )
	);
end MaximoComunDivisor;

architecture arch of MaximoComunDivisor is
	signal a: std_logic_vector( 10 downto 0 );
	signal b: std_logic_vector( n-1 downto 0 );
	signal r: std_logic_vector( n-1 downto 0 );
	type states is ( ereset, e1, e2, eamayor, ebmayor, e3, e4, efin );
	signal state: states;
begin
	mcd: process( reloj, reset )
	begin
		if reset = '1' then
			state <= ereset;
		elsif reloj'event and reloj = '1' then
			case state is
				when ereset =>
					fin <= '0';
					if pulsoa = '0' then
						state <= e1;
					end if;
				when e1 =>
					fin <= '0';
					a <= dato;
					salida <= dato;
					if pulsob = '0' then
						state <= e2;
					end if;
				when e2 =>
					b <= dato;
					salida <= dato;
					if a > b then
						state <= eamayor;
					elsif a < b then
						state <= ebmayor;
					else
						state <= efin;
					end if;
				when eamayor =>
					r <= std_logic_vector( unsigned( a ) - unsigned( b ) );
					state <= e3;
				when ebmayor =>
					r <= std_logic_vector( unsigned( b ) - unsigned( a ) );
					state <= e4;
				when e3 =>
					a <= r;
					if r > b then
						state <= eamayor;
					elsif r < b then
						state <= ebmayor;
					else
						state <= efin;
					end if;
				when e4 =>
					b <= r;
					if r > a then
						state <= ebmayor;
					elsif r < a then
						state <= eamayor;
					else
						state <= efin;
					end if;
				when efin =>
					salida <= a;
					fin <= '1';
					if pulsoa = '0' then
						state <= e1;
					else
						state <= efin;
					end if;
			end case;
		end if;
	end process;
end arch;
