import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonePresetsUpdateInput } from './zone-presets-update.input';
import { ZonePresetsWhereUniqueInput } from './zone-presets-where-unique.input';

@ArgsType()
export class UpdateOneZonePresetsArgs {

    @Field(() => ZonePresetsUpdateInput, {nullable:false})
    data!: ZonePresetsUpdateInput;

    @Field(() => ZonePresetsWhereUniqueInput, {nullable:false})
    where!: ZonePresetsWhereUniqueInput;
}
