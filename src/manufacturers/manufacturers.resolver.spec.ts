import { Test, TestingModule } from '@nestjs/testing';
import { ManufacturersResolver } from './manufacturers.resolver';
import { ManufacturersService } from './manufacturers.service';

describe('ManufacturersResolver', () => {
  let resolver: ManufacturersResolver;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ManufacturersResolver, ManufacturersService],
    }).compile();

    resolver = module.get<ManufacturersResolver>(ManufacturersResolver);
  });

  it('should be defined', () => {
    expect(resolver).toBeDefined();
  });
});
