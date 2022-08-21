import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonePresetsWhereInput } from './zone-presets-where.input';
import { Type } from 'class-transformer';
import { ZonePresetsOrderByWithRelationInput } from './zone-presets-order-by-with-relation.input';
import { ZonePresetsWhereUniqueInput } from './zone-presets-where-unique.input';
import { Int } from '@nestjs/graphql';
import { ZonePresetsScalarFieldEnum } from './zone-presets-scalar-field.enum';

@ArgsType()
export class FindFirstZonePresetsArgs {

    @Field(() => ZonePresetsWhereInput, {nullable:true})
    @Type(() => ZonePresetsWhereInput)
    where?: ZonePresetsWhereInput;

    @Field(() => [ZonePresetsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<ZonePresetsOrderByWithRelationInput>;

    @Field(() => ZonePresetsWhereUniqueInput, {nullable:true})
    cursor?: ZonePresetsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [ZonePresetsScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof ZonePresetsScalarFieldEnum>;
}
