import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MontageLayoutsWhereUniqueInput } from './montage-layouts-where-unique.input';
import { MontageLayoutsCreateInput } from './montage-layouts-create.input';
import { MontageLayoutsUpdateInput } from './montage-layouts-update.input';

@ArgsType()
export class UpsertOneMontageLayoutsArgs {

    @Field(() => MontageLayoutsWhereUniqueInput, {nullable:false})
    where!: MontageLayoutsWhereUniqueInput;

    @Field(() => MontageLayoutsCreateInput, {nullable:false})
    create!: MontageLayoutsCreateInput;

    @Field(() => MontageLayoutsUpdateInput, {nullable:false})
    update!: MontageLayoutsUpdateInput;
}
