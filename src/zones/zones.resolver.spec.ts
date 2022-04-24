import { Test, TestingModule } from '@nestjs/testing';
import { ZonesResolver } from './zones.resolver';
import { ZonesService } from './zones.service';

describe('ZonesResolver', () => {
  let resolver: ZonesResolver;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ZonesResolver, ZonesService],
    }).compile();

    resolver = module.get<ZonesResolver>(ZonesResolver);
  });

  it('should be defined', () => {
    expect(resolver).toBeDefined();
  });
});
