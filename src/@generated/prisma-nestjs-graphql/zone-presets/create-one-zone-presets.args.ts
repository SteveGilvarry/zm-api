import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonePresetsCreateInput } from './zone-presets-create.input';

@ArgsType()
export class CreateOneZonePresetsArgs {

    @Field(() => ZonePresetsCreateInput, {nullable:false})
    data!: ZonePresetsCreateInput;
}
