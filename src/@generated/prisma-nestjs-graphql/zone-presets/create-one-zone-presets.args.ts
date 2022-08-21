import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonePresetsCreateInput } from './zone-presets-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneZonePresetsArgs {

    @Field(() => ZonePresetsCreateInput, {nullable:false})
    @Type(() => ZonePresetsCreateInput)
    data!: ZonePresetsCreateInput;
}
