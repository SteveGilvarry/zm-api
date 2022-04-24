import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonePresetsCreateManyInput } from './zone-presets-create-many.input';

@ArgsType()
export class CreateManyZonePresetsArgs {

    @Field(() => [ZonePresetsCreateManyInput], {nullable:false})
    data!: Array<ZonePresetsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
