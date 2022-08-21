import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MontageLayoutsWhereUniqueInput } from './montage-layouts-where-unique.input';
import { Type } from 'class-transformer';
import { MontageLayoutsCreateInput } from './montage-layouts-create.input';
import { MontageLayoutsUpdateInput } from './montage-layouts-update.input';

@ArgsType()
export class UpsertOneMontageLayoutsArgs {

    @Field(() => MontageLayoutsWhereUniqueInput, {nullable:false})
    @Type(() => MontageLayoutsWhereUniqueInput)
    where!: MontageLayoutsWhereUniqueInput;

    @Field(() => MontageLayoutsCreateInput, {nullable:false})
    @Type(() => MontageLayoutsCreateInput)
    create!: MontageLayoutsCreateInput;

    @Field(() => MontageLayoutsUpdateInput, {nullable:false})
    @Type(() => MontageLayoutsUpdateInput)
    update!: MontageLayoutsUpdateInput;
}
